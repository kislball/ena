#!/bin/sh

ena compile ./std/base/**/*.ena ./std/vm/**/*.ena ./examples/$1.ena
ena run ./output.enair
rm output.enair
