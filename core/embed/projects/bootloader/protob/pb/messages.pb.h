/* Automatically generated nanopb header */
/* Generated by nanopb-0.4.5 */

#ifndef PB_MESSAGES_PB_H_INCLUDED
#define PB_MESSAGES_PB_H_INCLUDED
#include <pb.h>

#if PB_PROTO_HEADER_VERSION != 40
#error Regenerate this file with the current version of nanopb generator.
#endif

/* Enum definitions */
typedef enum _MessageType {
    MessageType_MessageType_Initialize = 0,
    MessageType_MessageType_Ping = 1,
    MessageType_MessageType_Success = 2,
    MessageType_MessageType_Failure = 3,
    MessageType_MessageType_WipeDevice = 5,
    MessageType_MessageType_FirmwareErase = 6,
    MessageType_MessageType_FirmwareUpload = 7,
    MessageType_MessageType_FirmwareRequest = 8,
    MessageType_MessageType_Features = 17,
    MessageType_MessageType_ButtonRequest = 26,
    MessageType_MessageType_ButtonAck = 27,
    MessageType_MessageType_GetFeatures = 55,
    MessageType_MessageType_UnlockBootloader = 96
} MessageType;

typedef enum _FailureType {
    FailureType_Failure_UnexpectedMessage = 1,
    FailureType_Failure_DataError = 3,
    FailureType_Failure_ActionCancelled = 4,
    FailureType_Failure_ProcessError = 9,
    FailureType_Failure_Busy = 15
} FailureType;

typedef enum _ButtonRequestType {
    ButtonRequestType_ButtonRequest_Other = 1
} ButtonRequestType;

/* Struct definitions */
typedef struct _ButtonAck {
    char dummy_field;
} ButtonAck;

typedef struct _GetFeatures {
    char dummy_field;
} GetFeatures;

typedef struct _Initialize {
    char dummy_field;
} Initialize;

typedef struct _UnlockBootloader {
    char dummy_field;
} UnlockBootloader;

typedef struct _WipeDevice {
    char dummy_field;
} WipeDevice;

typedef struct _ButtonRequest {
    bool has_code;
    ButtonRequestType code;
} ButtonRequest;

typedef struct _Failure {
    bool has_code;
    FailureType code;
    bool has_message;
    char message[256];
} Failure;

typedef PB_BYTES_ARRAY_T(20) Features_revision_t;
typedef struct _Features {
    bool has_vendor;
    char vendor[33];
    uint32_t major_version;
    uint32_t minor_version;
    uint32_t patch_version;
    bool has_bootloader_mode;
    bool bootloader_mode;
    bool has_device_id;
    char device_id[25];
    bool has_language;
    char language[17];
    bool has_label;
    char label[33];
    bool has_initialized;
    bool initialized;
    bool has_revision;
    Features_revision_t revision;
    bool has_firmware_present;
    bool firmware_present;
    bool has_model;
    char model[17];
    bool has_fw_major;
    uint32_t fw_major;
    bool has_fw_minor;
    uint32_t fw_minor;
    bool has_fw_patch;
    uint32_t fw_patch;
    bool has_fw_vendor;
    char fw_vendor[256];
    bool has_internal_model;
    char internal_model[17];
    bool has_unit_color;
    uint32_t unit_color;
    bool has_unit_btconly;
    bool unit_btconly;
    bool has_bootloader_locked;
    bool bootloader_locked;
    bool has_unit_packaging;
    uint32_t unit_packaging;
} Features;

typedef struct _FirmwareErase {
    bool has_length;
    uint32_t length;
} FirmwareErase;

typedef struct _FirmwareRequest {
    uint32_t offset;
    uint32_t length;
} FirmwareRequest;

typedef PB_BYTES_ARRAY_T(32) FirmwareUpload_hash_t;
typedef struct _FirmwareUpload {
    pb_callback_t payload;
    bool has_hash;
    FirmwareUpload_hash_t hash;
} FirmwareUpload;

typedef struct _Ping {
    bool has_message;
    char message[256];
} Ping;

typedef struct _Success {
    bool has_message;
    char message[256];
} Success;


/* Helper constants for enums */
#define _MessageType_MIN MessageType_MessageType_Initialize
#define _MessageType_MAX MessageType_MessageType_UnlockBootloader
#define _MessageType_ARRAYSIZE ((MessageType)(MessageType_MessageType_UnlockBootloader+1))

#define _FailureType_MIN FailureType_Failure_UnexpectedMessage
#define _FailureType_MAX FailureType_Failure_ProcessError
#define _FailureType_ARRAYSIZE ((FailureType)(FailureType_Failure_ProcessError+1))

#define _ButtonRequestType_MIN ButtonRequestType_ButtonRequest_Other
#define _ButtonRequestType_MAX ButtonRequestType_ButtonRequest_Other
#define _ButtonRequestType_ARRAYSIZE ((ButtonRequestType)(ButtonRequestType_ButtonRequest_Other+1))


#ifdef __cplusplus
extern "C" {
#endif

/* Initializer values for message structs */
#define Initialize_init_default                  {0}
#define GetFeatures_init_default                 {0}
#define WipeDevice_init_default                  {0}
#define Features_init_default                    {false, "", 0, 0, 0, false, 0, false, "", false, "", false, "", false, 0, false, {0, {0}}, false, 0, false, "", false, 0, false, 0, false, 0, false, "", false, "", false, 0, false, 0, false, 0, false, 0}
#define Ping_init_default                        {false, ""}
#define Success_init_default                     {false, ""}
#define Failure_init_default                     {false, _FailureType_MIN, false, ""}
#define ButtonRequest_init_default               {false, _ButtonRequestType_MIN}
#define ButtonAck_init_default                   {0}
#define FirmwareErase_init_default               {false, 0}
#define FirmwareRequest_init_default             {0, 0}
#define FirmwareUpload_init_default              {{{NULL}, NULL}, false, {0, {0}}}
#define UnlockBootloader_init_default            {0}
#define Initialize_init_zero                     {0}
#define GetFeatures_init_zero                    {0}
#define WipeDevice_init_zero                     {0}
#define Features_init_zero                       {false, "", 0, 0, 0, false, 0, false, "", false, "", false, "", false, 0, false, {0, {0}}, false, 0, false, "", false, 0, false, 0, false, 0, false, "", false, "", false, 0, false, 0, false, 0, false, 0}
#define Ping_init_zero                           {false, ""}
#define Success_init_zero                        {false, ""}
#define Failure_init_zero                        {false, _FailureType_MIN, false, ""}
#define ButtonRequest_init_zero                  {false, _ButtonRequestType_MIN}
#define ButtonAck_init_zero                      {0}
#define FirmwareErase_init_zero                  {false, 0}
#define FirmwareRequest_init_zero                {0, 0}
#define FirmwareUpload_init_zero                 {{{NULL}, NULL}, false, {0, {0}}}
#define UnlockBootloader_init_zero               {0}

/* Field tags (for use in manual encoding/decoding) */
#define ButtonRequest_code_tag                   1
#define Failure_code_tag                         1
#define Failure_message_tag                      2
#define Features_vendor_tag                      1
#define Features_major_version_tag               2
#define Features_minor_version_tag               3
#define Features_patch_version_tag               4
#define Features_bootloader_mode_tag             5
#define Features_device_id_tag                   6
#define Features_language_tag                    9
#define Features_label_tag                       10
#define Features_initialized_tag                 12
#define Features_revision_tag                    13
#define Features_firmware_present_tag            18
#define Features_model_tag                       21
#define Features_fw_major_tag                    22
#define Features_fw_minor_tag                    23
#define Features_fw_patch_tag                    24
#define Features_fw_vendor_tag                   25
#define Features_internal_model_tag              44
#define Features_unit_color_tag                  45
#define Features_unit_btconly_tag                46
#define Features_bootloader_locked_tag           49
#define Features_unit_packaging_tag              51
#define FirmwareErase_length_tag                 1
#define FirmwareRequest_offset_tag               1
#define FirmwareRequest_length_tag               2
#define FirmwareUpload_payload_tag               1
#define FirmwareUpload_hash_tag                  2
#define Ping_message_tag                         1
#define Success_message_tag                      1

/* Struct field encoding specification for nanopb */
#define Initialize_FIELDLIST(X, a) \

#define Initialize_CALLBACK NULL
#define Initialize_DEFAULT NULL

#define GetFeatures_FIELDLIST(X, a) \

#define GetFeatures_CALLBACK NULL
#define GetFeatures_DEFAULT NULL

#define WipeDevice_FIELDLIST(X, a) \

#define WipeDevice_CALLBACK NULL
#define WipeDevice_DEFAULT NULL

#define Features_FIELDLIST(X, a) \
X(a, STATIC,   OPTIONAL, STRING,   vendor,            1) \
X(a, STATIC,   REQUIRED, UINT32,   major_version,     2) \
X(a, STATIC,   REQUIRED, UINT32,   minor_version,     3) \
X(a, STATIC,   REQUIRED, UINT32,   patch_version,     4) \
X(a, STATIC,   OPTIONAL, BOOL,     bootloader_mode,   5) \
X(a, STATIC,   OPTIONAL, STRING,   device_id,         6) \
X(a, STATIC,   OPTIONAL, STRING,   language,          9) \
X(a, STATIC,   OPTIONAL, STRING,   label,            10) \
X(a, STATIC,   OPTIONAL, BOOL,     initialized,      12) \
X(a, STATIC,   OPTIONAL, BYTES,    revision,         13) \
X(a, STATIC,   OPTIONAL, BOOL,     firmware_present,  18) \
X(a, STATIC,   OPTIONAL, STRING,   model,            21) \
X(a, STATIC,   OPTIONAL, UINT32,   fw_major,         22) \
X(a, STATIC,   OPTIONAL, UINT32,   fw_minor,         23) \
X(a, STATIC,   OPTIONAL, UINT32,   fw_patch,         24) \
X(a, STATIC,   OPTIONAL, STRING,   fw_vendor,        25) \
X(a, STATIC,   OPTIONAL, STRING,   internal_model,   44) \
X(a, STATIC,   OPTIONAL, UINT32,   unit_color,       45) \
X(a, STATIC,   OPTIONAL, BOOL,     unit_btconly,     46) \
X(a, STATIC,   OPTIONAL, BOOL,     bootloader_locked,  49) \
X(a, STATIC,   OPTIONAL, UINT32,   unit_packaging,   51)
#define Features_CALLBACK NULL
#define Features_DEFAULT NULL

#define Ping_FIELDLIST(X, a) \
X(a, STATIC,   OPTIONAL, STRING,   message,           1)
#define Ping_CALLBACK NULL
#define Ping_DEFAULT (const pb_byte_t*)"\x0a\x00\x00"

#define Success_FIELDLIST(X, a) \
X(a, STATIC,   OPTIONAL, STRING,   message,           1)
#define Success_CALLBACK NULL
#define Success_DEFAULT (const pb_byte_t*)"\x0a\x00\x00"

#define Failure_FIELDLIST(X, a) \
X(a, STATIC,   OPTIONAL, UENUM,    code,              1) \
X(a, STATIC,   OPTIONAL, STRING,   message,           2)
#define Failure_CALLBACK NULL
#define Failure_DEFAULT (const pb_byte_t*)"\x08\x01\x00"

#define ButtonRequest_FIELDLIST(X, a) \
X(a, STATIC,   OPTIONAL, UENUM,    code,              1)
#define ButtonRequest_CALLBACK NULL
#define ButtonRequest_DEFAULT (const pb_byte_t*)"\x08\x01\x00"

#define ButtonAck_FIELDLIST(X, a) \

#define ButtonAck_CALLBACK NULL
#define ButtonAck_DEFAULT NULL

#define FirmwareErase_FIELDLIST(X, a) \
X(a, STATIC,   OPTIONAL, UINT32,   length,            1)
#define FirmwareErase_CALLBACK NULL
#define FirmwareErase_DEFAULT NULL

#define FirmwareRequest_FIELDLIST(X, a) \
X(a, STATIC,   REQUIRED, UINT32,   offset,            1) \
X(a, STATIC,   REQUIRED, UINT32,   length,            2)
#define FirmwareRequest_CALLBACK NULL
#define FirmwareRequest_DEFAULT NULL

#define FirmwareUpload_FIELDLIST(X, a) \
X(a, CALLBACK, REQUIRED, BYTES,    payload,           1) \
X(a, STATIC,   OPTIONAL, BYTES,    hash,              2)
#define FirmwareUpload_CALLBACK pb_default_field_callback
#define FirmwareUpload_DEFAULT NULL

#define UnlockBootloader_FIELDLIST(X, a) \

#define UnlockBootloader_CALLBACK NULL
#define UnlockBootloader_DEFAULT NULL

extern const pb_msgdesc_t Initialize_msg;
extern const pb_msgdesc_t GetFeatures_msg;
extern const pb_msgdesc_t WipeDevice_msg;
extern const pb_msgdesc_t Features_msg;
extern const pb_msgdesc_t Ping_msg;
extern const pb_msgdesc_t Success_msg;
extern const pb_msgdesc_t Failure_msg;
extern const pb_msgdesc_t ButtonRequest_msg;
extern const pb_msgdesc_t ButtonAck_msg;
extern const pb_msgdesc_t FirmwareErase_msg;
extern const pb_msgdesc_t FirmwareRequest_msg;
extern const pb_msgdesc_t FirmwareUpload_msg;
extern const pb_msgdesc_t UnlockBootloader_msg;

/* Defines for backwards compatibility with code written before nanopb-0.4.0 */
#define Initialize_fields &Initialize_msg
#define GetFeatures_fields &GetFeatures_msg
#define WipeDevice_fields &WipeDevice_msg
#define Features_fields &Features_msg
#define Ping_fields &Ping_msg
#define Success_fields &Success_msg
#define Failure_fields &Failure_msg
#define ButtonRequest_fields &ButtonRequest_msg
#define ButtonAck_fields &ButtonAck_msg
#define FirmwareErase_fields &FirmwareErase_msg
#define FirmwareRequest_fields &FirmwareRequest_msg
#define FirmwareUpload_fields &FirmwareUpload_msg
#define UnlockBootloader_fields &UnlockBootloader_msg

/* Maximum encoded size of messages (where known) */
/* FirmwareUpload_size depends on runtime parameters */
#define ButtonAck_size                           0
#define ButtonRequest_size                       2
#define Failure_size                             260
#define Features_size                            497
#define FirmwareErase_size                       6
#define FirmwareRequest_size                     12
#define GetFeatures_size                         0
#define Initialize_size                          0
#define Ping_size                                258
#define Success_size                             258
#define UnlockBootloader_size                    0
#define WipeDevice_size                          0

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif
