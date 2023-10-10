GET http://{{url}}/status/ping
HTTP 200

GET http://{{url}}/api/geocoding?location=Trondheim
HTTP 200

GET http://{{url}}/api/alerts
HTTP 200

GET http://{{url}}/api/nowcasts?location=Trondheim
HTTP 200

GET http://{{url}}/api/nowcasts?lat=63.4308&lon=10.4034
HTTP 200

GET http://{{url}}/api/nowcasts?location=Oslo
HTTP 200

GET http://{{url}}/api/nowcasts?location=Heimdal
HTTP 500
