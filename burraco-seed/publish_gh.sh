#!/bin/bash

cp *.html *.css ../../blofroth.github.io/burraco
cp index_gh.html ../../blofroth.github.io/burraco/index.html
cp pkg/* ../../blofroth.github.io/burraco/pkg

cd ../../blofroth.github.io/burraco
sed -i "s/BURRACO_VERSION/$(git rev-parse --short HEAD), $(date --rfc-3339=date)/g" ../../blofroth.github.io/burraco/credits.html
#git add .
#git commit -m "Publishing burraco from\n$(git log -1 --oneline)"
#git push