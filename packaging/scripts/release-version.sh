#!/usr/bin/env bash
set -euo pipefail

cargo pkgid -p pip-switch | sed 's/.*#//'
