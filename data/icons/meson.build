if get_option('profile') == 'development'
  install_data(
      'hicolor/com.gitlab.newsflash.Devel.svg',
      rename: '@0@.svg'.format(application_id),
      install_dir: datadir / 'icons' / 'hicolor' / 'scalable' / 'apps'
  )
else
  install_data(
      'hicolor/com.gitlab.newsflash.svg',
      rename: '@0@.svg'.format(application_id),
      install_dir: datadir / 'icons' / 'hicolor' / 'scalable' / 'apps'
  )
endif
install_data(
    'hicolor/com.gitlab.newsflash-symbolic.svg',
    rename: '@0@-symbolic.svg'.format(application_id),
    install_dir: datadir / 'icons' / 'hicolor' / 'symbolic' / 'apps'
)
