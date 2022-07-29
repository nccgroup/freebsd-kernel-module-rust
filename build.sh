#!/usr/bin/env sh

OBJECTDIR=/tmp/rustmodule
CURDIR=`pwd`
MODULE_NAME="bsd_rust"

if [ -d "${OBJECTDIR}" ]; then
	rm -rf "${OBJECTDIR}"
fi

mkdir "${OBJECTDIR}"



make clean && \
	cargo build && \
	cd "${OBJECTDIR}" && \
	ar -xv "${CURDIR}/target/x86_64-kernel-freebsd/debug/lib${MODULE_NAME}.a" && \
	cd "${CURDIR}" && \
	make OBJECTDIR="${OBJECTDIR}"
