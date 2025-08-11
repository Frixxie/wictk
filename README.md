# wictk (Weather is cool to know)

Wictk is a api wrapper around [Meterologisk instiutt](https://api.met.no) and [Openweathermap](https://openweathermap.com).
The main features is providing nowcasts and weather related alerts in norway

## Requirements

* Rust
* API key from Openweathermap

## How to run

To build
```sh
OPENWEATHERMAPAPIKEY=xxx cargo build
```

To run
```sh
OPENWEATHERMAPAPIKEY=xxx cargo run
```

## API Endpoints

### Lightning

#### Get Recent Lightning
Get recent lightning strikes with optional location and radius filtering:

```
# Get all recent lightning strikes from the last 24 hours
GET /api/recent_lightning

# Get lightning strikes within 100km of Oslo
GET /api/recent_lightning?location=Oslo&radius_km=100

# Get lightning strikes within 50km of specific coordinates
GET /api/recent_lightning?lat=63.4308&lon=10.4034&radius_km=50

# Get lightning strikes within default 50km radius of Trondheim
GET /api/recent_lightning?location=Trondheim
```

Parameters:
- `location`: City name (optional, alternative to lat/lon)
- `lat`: Latitude coordinate (optional, alternative to location)
- `lon`: Longitude coordinate (optional, alternative to location)  
- `radius_km`: Search radius in kilometers (optional, defaults to 50km when location is specified)

**Note**: When no location parameters are provided, returns all recent lightning strikes. When location is specified, filters results within the given radius.

### Other Endpoints

* `/api/alerts` - Weather alerts
* `/api/nowcasts` - Weather nowcasts
* `/api/geocoding` - Location geocoding
