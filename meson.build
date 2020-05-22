project(
  'newsflash', 'rust',
  version: '1.0.0',
  license: 'GPLv3',
)

i18n = import('i18n')
gnome = import('gnome')


dependency('glib-2.0', version: '>= 2.56')
dependency('gio-2.0', version: '>= 2.56')
dependency('gdk-pixbuf-2.0')
dependency('gtk+-3.0', version: '>= 3.24.7')

dependency('webkit2gtk-4.0', version: '>= 2.24.1')



cargo = find_program('cargo', required: true)
gresource = find_program('glib-compile-resources', required: true)
gschemas = find_program('glib-compile-schemas', required: true)
cargo_vendor = find_program('cargo-vendor', required: false)
cargo_script = find_program('build-aux/cargo.sh')



if get_option('profile') == 'development'
  profile = 'Devel'
  app_id_suffix = '.Devel'
  name_suffix = ' (Development)'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = 'RC1-@0@'.format (vcs_tag)
  endif
else
  profile = ''
  app_id_suffix = ''
  name_suffix = ''
  version_suffix = ''
endif

application_id = 'com.gitlab.newsflash@0@'.format(app_id_suffix)

newsflash_version = meson.project_version()
version_array = newsflash_version.split('.')
newsflash_major_version = version_array[0].to_int()
newsflash_minor_version = version_array[1].to_int()
newsflash_version_micro = version_array[2].to_int()

newsflash_prefix = get_option('prefix')
newsflash_bindir = join_paths(newsflash_prefix, get_option('bindir'))
newsflash_localedir = join_paths(newsflash_prefix, get_option('localedir'))

datadir = get_option('datadir')
podir = join_paths (meson.source_root (), 'po')

subdir('data')
subdir('po')

subdir('src')

meson.add_dist_script(
  'build-aux/vendor.sh',
  meson.source_root(),
  join_paths(meson.build_root(), 'meson-dist', meson.project_name() + '-' + newsflash_version)
)

meson.add_install_script('build-aux/meson_post_install.py')
