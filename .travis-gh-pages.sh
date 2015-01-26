#!/bin/sh

PROJECT_NAME=rust-websocket
CRATE_NAME=websocket

mv ./target/examples ./build-examples

echo "Generating documentation..."
PROJECT_VERSION=$(cargo doc | grep "Compiling ${CRATE_NAME} v" | sed 's/.*Compiling '"${CRATE_NAME}"' v\(.*\) .*/\1/')
echo "Done generating documentation (found version ${PROJECT_VERSION})"

echo "Running Autobahn TestSuite for client..."
wstest -m fuzzingserver -s ./autobahn/fuzzingserver.json & 
FUZZINGSERVER_PID=$!
sleep 10
./build-examples/autobahn-client
kill -9 ${FUZZINGSERVER_PID}
echo "Done running Autobahn TestSuite for client"

echo "Running Autobahn TestSuite for server..."
./build-examples/autobahn-server &
WSSERVER_PID=$!
sleep 10
wstest -m fuzzingclient -s ./autobahn/fuzzingclient.json
kill -9 ${WSSERVER_PID}
echo "Done running Autobahn TestSuite for server"

echo "Generating gh-pages..."

curl -sL https://github.com/${TRAVIS_REPO_SLUG}/archive/html.tar.gz | tar xz

cd ./${PROJECT_NAME}-html

find . -type f | xargs sed -i 's/<!--VERSION-->/'"${PROJECT_VERSION}"'/g'

mv ../target/doc ./doc
mv ../autobahn/server ./autobahn/server
mv ../autobahn/client ./autobahn/client

mv ./autobahn/server/index.json ./autobahn/server/index.temp
mv ./autobahn/client/index.json ./autobahn/client/index.temp

rm ./autobahn/server/*.json
rm ./autobahn/client/*.json

mv ./autobahn/server/index.temp ./autobahn/server/index.json
mv ./autobahn/client/index.temp ./autobahn/client/index.json

cd ../

echo "Done generating gh-pages"

echo "Pushing gh-pages"

ghp-import -n ./${PROJECT_NAME}-html

git push -fq https://${TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages