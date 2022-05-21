# Code to take accelerometer readings with the three visualiser axes each
# pointing down and turn it into an isometry rotation matrix

import numpy as np

# Input the raw accelerometer readings
viz_x = np.array([0.36152446, 8.118738, -5.5521536])
viz_y = np.array([-9.85214, -0.21787235, -0.20829555])
viz_z = np.array([0.320823, -5.303157, -8.616732])

# Normalise each on e
viz_x /= np.linalg.norm(viz_x)
viz_y /= np.linalg.norm(viz_y)
viz_z /= np.linalg.norm(viz_z)

# Orthogonalise by replacing one with a cross of the other two.  Best option
# found by trial and error
new_y = np.cross(viz_x, viz_z)

# Turn them all into actual vectors
vec_x = np.array([viz_x]).T
vec_y = np.array([new_y]).T
vec_z = np.array([viz_z]).T

# Stack up horizontally to make the rotation matrix
rot = np.hstack((vec_x, vec_y, vec_z))

print(rot)
