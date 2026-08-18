"""Fix - single position fix"""
import numpy


class Fix:
    """Single position fix from IGC track"""

    def __init__(self, data, index: int | None = None):
        """Wrap a single numpy structured array element"""
        self._data = data
        self._index = index

    @property
    def index(self) -> int | None:
        """Position in the track it was read from if any"""
        return self._index

    @property
    def latitude(self) -> float:
        """Latitude in decimal degrees"""
        return float(self._data['latitude'])

    @property
    def longitude(self) -> float:
        """Longitude in decimal degrees"""
        return float(self._data['longitude'])

    @property
    def baro_altitude(self) -> int:
        """Barometric altitude in meters"""
        return int(self._data['baro_altitude'])

    @property
    def gnss_altitude(self) -> int:
        """GNSS altitude in meters"""
        return int(self._data['gnss_altitude'])

    @property
    def timestamp(self) -> int:
        """Timestamp in seconds since midnight"""
        return int(self._data['timestamp'])

    def __eq__(self, other) -> bool:
        """Compare Fix objects by their underlying data"""
        if not isinstance(other, Fix):
            return False
        return bool(numpy.array_equal(self._data, other._data))

    def __repr__(self) -> str:
        return f"Fix(lat={self.latitude:.6f}, lon={self.longitude:.6f}, alt={self.baro_altitude}m)"
