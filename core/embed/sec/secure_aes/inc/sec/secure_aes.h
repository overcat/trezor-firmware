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

#ifdef KERNEL
#include <sys/applet.h>
#endif

// only some of the keys are supported depending on execution environment
typedef enum {
  SECURE_AES_KEY_DHUK_SP,  // secure-privileged
  SECURE_AES_KEY_BHK,
  SECURE_AES_KEY_XORK_SP,  // secure-privileged
  SECURE_AES_KEY_XORK_SN,  // secure-nonprivileged
} secure_aes_keysel_t;

// Initializes secure AES module
secbool secure_aes_init(void);

// Sets the applet to be used for AES operation
// with unprivileged key (XORK_SN).
#ifdef KERNEL
void secure_aes_set_applet(applet_t* applet);
#endif

// Deinitializes secure AES module
void secure_aes_deinit(void);

// Encrypts a block of data using AES-256 ECB and HW key (DHUK, BHK or XORK)
// For optimal speed input and output should be aligned to 32 bits, size is in
// bytes
secbool secure_aes_ecb_encrypt_hw(const uint8_t* input, size_t size,
                                  uint8_t* output, secure_aes_keysel_t key);

// Decrypts a block of data using AES-256 ECB and HW key (DHUK, BHK or XORK)
// For optimal speed input and output should be aligned to 32 bits, size is in
// bytes
secbool secure_aes_ecb_decrypt_hw(const uint8_t* input, size_t size,
                                  uint8_t* output, secure_aes_keysel_t key);
