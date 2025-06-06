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

#pragma once

#include <trezor_types.h>

#include <sys/system.h>

#ifdef SECMON

// Checks if the non-secure code has read access to the
// given memory range.
bool probe_read_access(const void *addr, size_t len);

// Checks if the non-secure code has write access to the
// given memory range.
bool probe_write_access(void *addr, size_t len);

// Checks if the provided address is in non-secure address range
bool probe_execute_access(const void *addr);

// Exits the current application task with an fatal error
// with the message "Access violation".
#define apptask_access_violation()                             \
  do {                                                         \
    system_exit_fatal("Access violation", __FILE__, __LINE__); \
  } while (0)

#endif  // SECMON
