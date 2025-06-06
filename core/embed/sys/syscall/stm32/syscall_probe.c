/*
 * This file is part of the Trezor project, https://trezor.io/
 *
 * Copyright (c) SatoshiLabs
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

// Turning off the stack protector for this file improves
// the performance of syscall dispatching.
#pragma GCC optimize("no-stack-protector")

#include <trezor_model.h>
#include <trezor_rtl.h>

#include <sys/applet.h>

#include "syscall_probe.h"

#ifdef KERNEL

static inline bool inside_area(const void *addr, size_t len,
                               const mpu_area_t *area) {
  return ((uintptr_t)addr >= area->start) &&
         ((uintptr_t)addr + len <= area->start + area->size);
}

bool probe_read_access(const void *addr, size_t len) {
  applet_t *applet = syscall_get_context();

  if (applet == NULL) {
    return false;
  }

  if (addr == NULL) {
    return true;
  }

  // Address overflow check
  if ((uintptr_t)addr + len < (uintptr_t)addr) {
    return false;
  }

  if (inside_area(addr, len, &applet->layout.data1)) {
    return true;
  }

  if (inside_area(addr, len, &applet->layout.data2)) {
    return true;
  }

#ifdef FRAMEBUFFER
  if (mpu_inside_active_fb(addr, len)) {
    return true;
  }
#endif

  if (inside_area(addr, len, &applet->layout.code1)) {
    return true;
  }

  if (inside_area(addr, len, &applet->layout.code2)) {
    return true;
  }

  static const mpu_area_t assets = {
      .start = ASSETS_START,
      .size = ASSETS_MAXSIZE,
  };

  if (inside_area(addr, len, &assets)) {
    return true;
  }

  return false;
}

bool probe_write_access(void *addr, size_t len) {
  applet_t *applet = syscall_get_context();

  if (applet == NULL) {
    return false;
  }

  if (addr == NULL) {
    return true;
  }

  // Address overflow check
  if ((uintptr_t)addr + len < (uintptr_t)addr) {
    return false;
  }

  if (inside_area(addr, len, &applet->layout.data1)) {
    return true;
  }

  if (inside_area(addr, len, &applet->layout.data2)) {
    return true;
  }

#ifdef FRAMEBUFFER
  if (mpu_inside_active_fb(addr, len)) {
    return true;
  }
#endif

  return false;
}

void handle_access_violation(const char *file, int line) {
  static const char *msg = "Access violation";
  applet_t *applet = syscall_get_context();
  systask_t *task = applet != NULL ? &applet->task : systask_active();
  systask_exit_fatal(task, msg, strlen(msg), file, strlen(file), line);
}

#endif  // KERNEL
