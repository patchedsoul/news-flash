#!/usr/bin/env python3

import os
import subprocess

install_prefix = os.environ['MESON_INSTALL_PREFIX']
icondir = os.path.join(install_prefix, 'share', 'icons', 'hicolor')
schemadir = os.path.join(install_prefix, 'share', 'glib-2.0', 'schemas')
appsdir = os.path.join(install_prefix, 'share', 'applications')

if not os.environ.get('DESTDIR'):
  print('Updating icon cache...')
  subprocess.call(['gtk-update-icon-cache', '-f', '-t', icondir])

  print('Updating desktop database...')
  subprocess.call(['update-desktop-database', appsdir])
