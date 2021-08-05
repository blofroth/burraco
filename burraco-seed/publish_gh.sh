#!/bin/bash

cp *.html *.css ../../blofroth.github.io/burraco
cp index_gh.html ../../blofroth.github.io/burraco/index.html
cp pkg/* ../../blofroth.github.io/burraco/pkg
GIT_HASH="$(git rev-parse --short HEAD)"
GIT_LOG="$(git log -1 --oneline)"
cd ../../blofroth.github.io/burraco
sed -i "s/BURRACO_VERSION/${GIT_HASH}, $(date --rfc-3339=date)/g" ../../blofroth.github.io/burraco/index.html
git add .
git commit -m "Publishing burraco from\n${GIT_LOG}"
git push
