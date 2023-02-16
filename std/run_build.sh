#!/bin/sh
# temp file
ena compile ./base/**/*.ena ./base/*.ena ./vm/**/*.ena ./vm/*.ena
ena run output.enair
