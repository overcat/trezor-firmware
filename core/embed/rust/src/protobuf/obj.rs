use core::convert::TryFrom;

use crate::{
    error::Error,
    micropython::{
        dict::Dict,
        ffi,
        gc::Gc,
        macros::{obj_fn_1, obj_fn_2, obj_fn_3, obj_module, obj_type},
        map::Map,
        module::Module,
        obj::{Obj, ObjBase},
        qstr::Qstr,
        typ::Type,
        util,
    },
};

use super::{
    decode::{protobuf_decode, Decoder},
    defs::{find_name_by_msg_offset, get_msg, MsgDef},
    encode::{protobuf_encode, protobuf_len},
};

#[repr(C)]
pub struct MsgObj {
    base: ObjBase,
    map: Map,
    msg_wire_id: Option<u16>,
    msg_offset: u16,
}

impl MsgObj {
    pub fn alloc_with_capacity(capacity: usize, msg: &MsgDef) -> Result<Gc<Self>, Error> {
        Gc::new(Self {
            base: Self::obj_type().as_base(),
            map: Map::with_capacity(capacity)?,
            msg_wire_id: msg.wire_id,
            msg_offset: msg.offset,
        })
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn map_mut(&mut self) -> &mut Map {
        &mut self.map
    }

    pub fn def(&self) -> MsgDef {
        unsafe { get_msg(self.msg_offset) }
    }

    fn obj_type() -> &'static Type {
        static TYPE: Type = obj_type! {
            name: Qstr::MP_QSTR_Msg,
            attr_fn: msg_obj_attr,
        };
        &TYPE
    }
}

impl MsgObj {
    fn getattr(&self, attr: Qstr) -> Result<Obj, Error> {
        if let Ok(obj) = self.map.get(attr) {
            // Message field was found, return its value.
            return Ok(obj);
        }

        // Built-in attribute.
        match attr {
            Qstr::MP_QSTR_MESSAGE_WIRE_TYPE => {
                // Return the wire ID of this message def, or None if not set.
                Ok(self.msg_wire_id.map_or_else(Obj::const_none, Into::into))
            }
            Qstr::MP_QSTR_MESSAGE_NAME => {
                // Return the QSTR name of this message def.
                let name = Qstr::from_u16(
                    find_name_by_msg_offset(self.msg_offset)
                        .ok_or_else(|| Error::KeyError(self.msg_offset.into()))?,
                );
                Ok(name.into())
            }
            Qstr::MP_QSTR___dict__ => {
                // Conversion to dict. Allocate a new dict object with a copy of our map
                // and return it. This is a bit different from how uPy does it now, because
                // we're returning a mutable dict.
                Ok(Gc::new(Dict::with_map(self.map.try_clone()?))?.into())
            }
            _ => Err(Error::AttributeError(attr)),
        }
    }

    fn setattr(&mut self, attr: Qstr, value: Obj) -> Result<(), Error> {
        if value.is_null() {
            // Null value means a delattr operation, reject.
            return Err(Error::TypeError);
        }

        if self.map.contains_key(attr) {
            self.map.set(attr, value)?;
            Ok(())
        } else {
            Err(Error::AttributeError(attr))
        }
    }
}

impl From<Gc<MsgObj>> for Obj {
    fn from(value: Gc<MsgObj>) -> Self {
        // SAFETY:
        //  - `value` is GC-allocated.
        //  - `value` is `repr(C)`.
        //  - `value` has a `base` as the first field with the correct type.
        unsafe { Self::from_ptr(Gc::into_raw(value).cast()) }
    }
}

impl TryFrom<Obj> for Gc<MsgObj> {
    type Error = Error;

    fn try_from(value: Obj) -> Result<Self, Self::Error> {
        if MsgObj::obj_type().is_type_of(value) {
            // SAFETY: We assume that if `value` is an object pointer with the correct type,
            // it is always GC-allocated.
            let this = unsafe { Gc::from_raw(value.as_ptr().cast()) };
            Ok(this)
        } else {
            Err(Error::TypeError)
        }
    }
}

unsafe extern "C" fn msg_obj_attr(self_in: Obj, attr: ffi::qstr, dest: *mut Obj) {
    let block = || {
        let mut this = Gc::<MsgObj>::try_from(self_in)?;
        let attr = Qstr::from_u16(attr as _);

        unsafe {
            if dest.read().is_null() {
                // Load attribute.
                dest.write(this.getattr(attr)?);
            } else {
                let value = dest.offset(1).read();
                // Store attribute.
                Gc::as_mut(&mut this).setattr(attr, value)?;
                dest.write(Obj::const_null());
            }
            Ok(())
        }
    };
    unsafe { util::try_or_raise(block) }
}

#[repr(C)]
pub struct MsgDefObj {
    base: ObjBase,
    def: MsgDef,
}

impl MsgDefObj {
    pub fn alloc(def: MsgDef) -> Result<Gc<Self>, Error> {
        let this = Gc::new(Self {
            base: Self::obj_type().as_base(),
            def,
        })?;
        Ok(this)
    }

    pub fn msg(&self) -> &MsgDef {
        &self.def
    }

    fn obj_type() -> &'static Type {
        static TYPE: Type = obj_type! {
            name: Qstr::MP_QSTR_MsgDef,
            attr_fn: msg_def_obj_attr,
            call_fn: msg_def_obj_call,
        };
        &TYPE
    }
}

impl From<Gc<MsgDefObj>> for Obj {
    fn from(value: Gc<MsgDefObj>) -> Self {
        // SAFETY:
        //  - `value` is GC-allocated.
        //  - `value` is `repr(C)`.
        //  - `value` has a `base` as the first field with the correct type.
        unsafe { Self::from_ptr(Gc::into_raw(value).cast()) }
    }
}

impl TryFrom<Obj> for Gc<MsgDefObj> {
    type Error = Error;

    fn try_from(value: Obj) -> Result<Self, Self::Error> {
        if MsgDefObj::obj_type().is_type_of(value) {
            // SAFETY: We assume that if `value` is an object pointer with the correct type,
            // it is always GC-allocated.
            let this = unsafe { Gc::from_raw(value.as_ptr().cast()) };
            Ok(this)
        } else {
            Err(Error::TypeError)
        }
    }
}

unsafe extern "C" fn msg_def_obj_attr(self_in: Obj, attr: ffi::qstr, dest: *mut Obj) {
    let block = || {
        let this = Gc::<MsgDefObj>::try_from(self_in)?;
        let attr = Qstr::from_u16(attr as _);

        let arg = unsafe { dest.read() };
        if !arg.is_null() {
            // Null destination would mean a `setattr`.
            return Err(Error::TypeError);
        }

        match attr {
            Qstr::MP_QSTR_MESSAGE_NAME => {
                // Return the QSTR name of this message def.
                let name = Qstr::from_u16(unwrap!(find_name_by_msg_offset(this.def.offset)));
                unsafe {
                    dest.write(name.into());
                };
            }
            Qstr::MP_QSTR_MESSAGE_WIRE_TYPE => {
                // Return the wire type of this message def.
                let wire_id_obj = this.def.wire_id.map_or_else(Obj::const_none, Into::into);
                unsafe {
                    dest.write(wire_id_obj);
                };
            }
            Qstr::MP_QSTR_is_type_of => {
                // Return the `is_type_of` bound method:
                // dest[0] = function_obj
                // dest[1] = self
                unsafe {
                    dest.write(MSG_DEF_OBJ_IS_TYPE_OF_OBJ.as_obj());
                    dest.offset(1).write(self_in);
                }
            }
            _ => {
                return Err(Error::AttributeError(attr));
            }
        }
        Ok(())
    };
    unsafe { util::try_or_raise(block) }
}

unsafe extern "C" fn msg_def_obj_call(
    self_in: Obj,
    n_args: usize,
    n_kw: usize,
    args: *const Obj,
) -> Obj {
    let block = |_args: &[Obj], kwargs: &Map| {
        let this = Gc::<MsgDefObj>::try_from(self_in)?;
        let decoder = Decoder {
            enable_experimental: true,
        };
        let obj = decoder.message_from_values(kwargs, this.msg())?;
        Ok(obj)
    };
    unsafe { util::try_with_args_and_kwargs_inline(n_args, n_kw, args, block) }
}

unsafe extern "C" fn msg_def_obj_is_type_of(self_in: Obj, obj: Obj) -> Obj {
    let block = || {
        let this = Gc::<MsgDefObj>::try_from(self_in)?;
        let msg = Gc::<MsgObj>::try_from(obj);
        match msg {
            Ok(msg) if msg.msg_offset == this.def.offset => Ok(Obj::const_true()),
            _ => Ok(Obj::const_false()),
        }
    };
    unsafe { util::try_or_raise(block) }
}

static MSG_DEF_OBJ_IS_TYPE_OF_OBJ: ffi::mp_obj_fun_builtin_fixed_t =
    obj_fn_2!(msg_def_obj_is_type_of);

#[no_mangle]
pub extern "C" fn protobuf_debug_msg_type() -> &'static Type {
    MsgObj::obj_type()
}

#[no_mangle]
pub extern "C" fn protobuf_debug_msg_def_type() -> &'static Type {
    MsgDefObj::obj_type()
}

pub extern "C" fn protobuf_type_for_name(name: Obj) -> Obj {
    let block = || {
        let name = Qstr::try_from(name)?;
        let def = MsgDef::for_name(name.to_u16()).ok_or_else(|| Error::KeyError(name.into()))?;
        let obj = MsgDefObj::alloc(def)?.into();
        Ok(obj)
    };
    unsafe { util::try_or_raise(block) }
}

pub extern "C" fn protobuf_type_for_wire(enum_name: Obj, wire_id: Obj) -> Obj {
    let block = || {
        let wire_id = u16::try_from(wire_id)?;
        let enum_name = Qstr::try_from(enum_name)?;
        let def = MsgDef::for_wire_id(enum_name.to_u16(), wire_id)
            .ok_or_else(|| Error::KeyError(wire_id.into()))?;
        let obj = MsgDefObj::alloc(def)?.into();
        Ok(obj)
    };
    unsafe { util::try_or_raise(block) }
}

#[no_mangle]
pub static mp_module_trezorproto: Module = obj_module! {
    /// from typing_extensions import Self
    ///
    /// # XXX
    /// # Note that MessageType "subclasses" are not true subclasses, but instead instances
    /// # of the built-in metaclass MsgDef. MessageType instances are in fact instances of
    /// # the built-in type Msg. That is why isinstance checks do not work, and instead the
    /// # MessageTypeSubclass.is_type_of() method must be used.
    ///
    /// class MessageType:
    ///     MESSAGE_NAME: ClassVar[str] = "MessageType"
    ///     MESSAGE_WIRE_TYPE: ClassVar[int | None] = None
    ///
    ///     @classmethod
    ///     def is_type_of(cls: type[Self], msg: "MessageType") -> TypeGuard[Self]:
    ///         """Identify if the provided message belongs to this type."""
    ///
    /// mock:global
    /// T = TypeVar("T", bound=MessageType)

    Qstr::MP_QSTR___name__ => Qstr::MP_QSTR_trezorproto.to_obj(),

    /// def type_for_name(name: str) -> type[MessageType]:
    ///     """Find the message definition for the given protobuf name."""
    Qstr::MP_QSTR_type_for_name => obj_fn_1!(protobuf_type_for_name).as_obj(),

    /// def type_for_wire(enum_name: str, wire_id: int) -> type[MessageType]:
    ///     """Find the message definition for the given wire enum name and
    ///     wire type (numeric identifier)."""
    Qstr::MP_QSTR_type_for_wire => obj_fn_2!(protobuf_type_for_wire).as_obj(),

    /// def decode(
    ///     buffer: bytes,
    ///     msg_type: type[T],
    ///     enable_experimental: bool,
    /// ) -> T:
    ///     """Decode data in the buffer into the specified message type."""
    Qstr::MP_QSTR_decode => obj_fn_3!(protobuf_decode).as_obj(),

    /// def encoded_length(msg: MessageType) -> int:
    ///     """Calculate length of encoding of the specified message."""
    Qstr::MP_QSTR_encoded_length => obj_fn_1!(protobuf_len).as_obj(),

    /// def encode(buffer: bytearray | memoryview, msg: MessageType) -> int:
    ///     """Encode the message into the specified buffer. Return length of
    ///     encoding."""
    Qstr::MP_QSTR_encode => obj_fn_2!(protobuf_encode).as_obj()
};
