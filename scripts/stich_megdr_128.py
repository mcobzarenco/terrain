import os
from collections import namedtuple

import numpy as np

X_STEP = 11520
Y_STEP = 5632

PREFIX = 'megr'
LATITUDE_STEPS = ['88n', '44n', '00n', '44s']
LONGITUDE_STEPS = ['000', '090', '180', '270']
SUFFIX = 'hb.img'

output = np.zeros((Y_STEP * 4, X_STEP * 4),
                  dtype='>i2')

for lat_index, lat_step in enumerate(LATITUDE_STEPS):
    for long_index, long_step in enumerate(LONGITUDE_STEPS):
        filename = PREFIX + lat_step + long_step + SUFFIX
        X = np.fromfile(filename, dtype='>i2').reshape((Y_STEP, X_STEP))
        output[lat_index * Y_STEP:(lat_index + 1) * Y_STEP,
               long_index * X_STEP:(long_index + 1) * X_STEP] = X
        print((lat_index * Y_STEP, (lat_index + 1) * Y_STEP,
               long_index * X_STEP, (long_index + 1) * X_STEP),
              filename, X.shape)

print('Saving heightmap of size {} with values in [{}, {}].'
      .format(output.shape, output.min(), output.max()))
output.tofile('megdr-128-stiched.img')

# output = np.zeros((5760, X_STEP),
#                   dtype='>i2')
# X = np.fromfile("/home/marius/w/terrain/assets/megr90n000fb.img",
#                 dtype='>i2').reshape((5760, X_STEP))
# output[:, :] = X
# output.tofile('stiched.img')
