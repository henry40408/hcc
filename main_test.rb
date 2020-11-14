require 'minitest/autorun'
require_relative 'main'

class TestMain < Minitest::Test
  def test_check_ssl_certificate
    r = check_ssl_certificate('example.com')
    assert r[:ok]
    assert r[:hostname] == 'example.com'
    assert r[:days].is_a?(Integer)

    refute check_ssl_certificate('expired.badssl.com')[:ok]
  end

  def test_check_ssl_certificate!
    assert_raises(OpenSSL::SSL::SSLError) do
      check_ssl_certificate!('expired.badssl.com')
    end
  end
end
