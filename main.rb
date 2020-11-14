require 'net/http'
require 'time_calc'

def check_ssl_certificate!(hostname, days: 7)
  not_after = TimeCalc.now.+(days, :days)

  cert = Net::HTTP.start(hostname, '443', use_ssl: true) { |http| http.peer_cert }
  diff = TimeCalc.(cert.not_after) - not_after
  {
    ok: diff.seconds >= 0,
    days: diff.days,
    seconds: diff.seconds,
  }
end

def check_ssl_certificate(hostname, days: 7)
  begin
    check_ssl_certificate!(hostname, days: days)
  rescue OpenSSL::SSL::SSLError
    {
      ok: false
    }
  end
end
