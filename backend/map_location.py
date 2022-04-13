# Bottom left of map is:
# Latitude:52°2'20"N Longitude:2°22'50"W
# 52.038889, -2.380556
#
# Top right of map is:
# Latitude:52°2'39"N Longitude:2°22'26"W
# 52.044046,-2.374030
#
# Base map is 994 pixels wide and 1275 pixels tall


def latlong_to_pix(longitude, latitude):
    x_pix_per_degree = 994 / (-2.374030 - -2.380556)
    y_pix_per_degree = 1275 / (52.038889 - 52.044046)
    x_degree_offset = -2.380556
    y_degree_offset = 52.044046

    x_pix = (longitude - x_degree_offset) * x_pix_per_degree
    y_pix = (latitude - y_degree_offset) * y_pix_per_degree

    return x_pix, y_pix


# Test: The triangle next to first aid is at:
latitude = 52.04165543846885
longitude = -2.377864785129338
print(latlong_to_pix(longitude, latitude))
