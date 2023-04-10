PAGES_PATH="target/gh-pages"
DEBIAN_PACKAGE_PATH="target/debian"

mkdir $PAGES_PATH 2>/dev/null

cp ${DEBIAN_PACKAGE_PATH}/*.deb $PAGES_PATH/

dpkg-scanpackages --multiversion $PAGES_PATH | gzip -9c >$PAGES_PATH/Packages.gz
