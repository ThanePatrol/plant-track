
docker run --rm -d -p 127.0.0.1:3000:3000 --env-file ./.env $(docker build -q .)
