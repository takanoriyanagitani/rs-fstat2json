#!/bin/sh

ls -1 |
	./rs-fstat2json |
	jq -c
