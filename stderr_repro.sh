#!/bin/bash
mv $1/TEST_BUILD $1/BUILD
./pants --enable-pantsd --build-file-imports=warn run "${1}:hello"
mv $1/BUILD $1/TEST_BUILD
