PAGES_PATH="target/gh-pages"
PPA_PATH="$PAGES_PATH/deb"
DEBIAN_PACKAGE_PATH="target/debian"
EMAIL="git@magierdinge.de"

mkdir -p $PPA_PATH 2>/dev/null

cp ${DEBIAN_PACKAGE_PATH}/*.deb $PPA_PATH/
cp ppa-deployment-key.pub $PPA_PATH/KEY.gpg

cd $PPA_PATH || exit


dpkg-scanpackages --multiversion . > Packages
gzip -k -f Packages

apt-ftparchive release . > Release

gpg --default-key "${EMAIL}" --clearsign -o - Release > InRelease
gpg --default-key "${EMAIL}" -abs -o - Release > Release.gpg
gpg --armor --export "${EMAIL}" > KEY.gpg
