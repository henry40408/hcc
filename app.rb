require 'sinatra'
require 'sinatra/json'
require 'sinatra/reloader' if development?

require_relative 'main'

get '/' do
  json status: 'OK'
end

get '/:hostname' do
  json check_ssl_certificate(params[:hostname])
end

get '/:hostname/:days' do
  json check_ssl_certificate(params[:hostname], days: params[:days].to_i)
end
