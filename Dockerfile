FROM ruby:2.7.1-alpine

COPY . .

RUN apk add --no-cache g++ musl-dev make && \
    bundle && \
    apk del g++ musl-dev make

CMD bundle exec rackup -o 0.0.0.0
