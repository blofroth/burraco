#!/bin/bash

cp *.html *.css ../../blofroth.github.io/burraco
cp index_gh.html ../../blofroth.github.io/burraco/index.html
cp pkg/* ../../blofroth.github.io/burraco/pkg

cd ../../blofroth.github.io/burraco
git add .
git commit -m "Publishing burraco from\n$(git log -1 --oneline)"
git push