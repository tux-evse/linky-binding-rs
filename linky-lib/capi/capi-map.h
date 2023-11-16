/*
 * Copyright (C) 2015-2022 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include <errno.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <termios.h>
#include <unistd.h>


// open flags
const int TTY_O_NOCTTY= O_NOCTTY;
const int TTY_O_NDELAY= O_NDELAY;
const int TTY_O_RDWR= O_RDWR;
const int TTY_O_RDONLY= O_RDONLY;
const int TTY_O_SYNC= O_SYNC;

// control flags (termio c_iflag)
const uint TIO_ICRNL= ICRNL;
const uint TIO_IGNCR= IGNCR;
const uint TIO_IGNPAR= IGNPAR;
const uint TIO_INPCK= INPCK;
const uint TIO_PARMRK= PARMRK;
const uint TIO_IGNBRK= IGNBRK;
const uint TIO_ISIG= ISIG;


// speed selection (termio speed)
const uint TIO_B1200= B1200;
const uint TIO_B2400= B2400;
const uint TIO_B9600= B9600;
const uint TIO_B19200= B19200;
const uint TIO_B38400= B38400;

// control bits  (termio c_cflags)
const uint TCF_CS7= CS7;
const uint TCF_CS8= CS8;
const uint TCF_PARENB= PARENB;
const uint TCF_PARODD= PARODD;
const uint TCF_CSTOPB= CSTOPB;
const uint TCF_CRTSCTS= CRTSCTS;
const uint TCF_CLOCAL= CLOCAL;

// control bits iflags
const uint TIF_IGNPAR= IGNPAR;
const uint TIF_IGNBRK= IGNBRK;
const uint TIF_INPCK= INPCK;
const uint TIF_INLCR= INLCR;
const uint TIF_IGNCR= IGNCR;
const uint TIF_IUCLC= IUCLC;
const uint TIF_IUTF8= IUTF8;
const uint TIF_ICRNL= ICRNL;

// local flags (termios c_lflags)
const uint TIO_ICANON= ICANON; // read line per line
const uint TIO_XCASE= XCASE; // read line per line

// attribute selection (tcsetattr
const uint TIO_TCSANOW= TCSANOW; // change attribute now

// line control (tcflush)
const int TIO_TCIOFLUSH= TCIOFLUSH; // flush pending input/oputput
const uint TIO_VMIN= VMIN; // Minimum number of characters for non canonical read (MIN).
