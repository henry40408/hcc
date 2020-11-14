FROM ruby:2.7.1-alpine

COPY Gemfile .
COPY Gemfile.lock .

RUN apk add --no-cache g++ musl-dev make && \
    bundle && \
    apk del g++ musl-dev make

COPY . .

EXPOSE 9292

CMD bundle exec rackup -o 0.0.0.0
