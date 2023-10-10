GET http://localhost:5000/status/ping
HTTP 200

GET http://localhost:5000/api/geocoding?location=Trondheim
HTTP 200

GET http://localhost:5000/api/alerts
HTTP 200

GET http://localhost:5000/api/nowcasts?location=Trondheim
HTTP 200

GET http://localhost:5000/api/nowcasts?lat=63.4308&lon=10.4034
HTTP 200

GET http://localhost:5000/api/nowcasts?location=Oslo
HTTP 200

GET http://localhost:5000/api/nowcasts?location=Heimdal
HTTP 500
