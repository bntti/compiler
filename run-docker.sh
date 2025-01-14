#!/bin/bash
docker build -t compiler .
docker run --rm --init -p 3000:3000 --name compiler compiler
