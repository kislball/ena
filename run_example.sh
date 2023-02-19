#!/bin/sh

ena compile "./std/base/**/*.ena ./std/vm/**/*.ena" -o ./std.enair
ena compile ./examples/$1.ena -o ./main.enair
ena link ./main.enair ./std.enair -o output.enair
ena run ./output.enair
rm output.enair main.enair std.enair
