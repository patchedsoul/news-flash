global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('VERSION', newsflash_version + version_suffix)
global_conf.set_quoted('PROFILE', profile)
config_rs = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)

run_command(
  'cp',
  config_rs,
  meson.current_source_dir(),
  check: true
)

newsflash_sources = files(
  'article_list/models/article.rs',
  'article_list/models/article_update_msg.rs',
  'article_list/models/change_set.rs',
  'article_list/models/error.rs',
  'article_list/models/mod.rs',
  'article_list/article_row.rs',
  'article_list/mod.rs',
  'article_list/single.rs',
  'article_view/models/mod.rs',
  'article_view/mod.rs',
  'article_view/progress_overlay.rs',
  'article_view/url_overlay.rs',
  'color/error.rs',
  'color/mod.rs',
  'content_page/content_header.rs',
  'content_page/header_selection.rs',
  'content_page/error.rs',
  'content_page/mod.rs',
  'discover/mod.rs',
  'discover/related_topic_row.rs',
  'discover/search_item_row.rs',
  'login_screen/mod.rs',
  'login_screen/password_login.rs',
  'login_screen/web_login.rs',
  'login_screen/error.rs',
  'settings/article_list.rs',
  'settings/article_view.rs',
  'settings/dialog.rs',
  'settings/error.rs',
  'settings/general.rs',
  'settings/keybinding_editor.rs',
  'settings/keybindings.rs',
  'settings/mod.rs',
  'settings/theme_chooser.rs',
  'sidebar/feed_list/models/category.rs',
  'sidebar/feed_list/models/change_set.rs',
  'sidebar/feed_list/models/dnd_action.rs',
  'sidebar/feed_list/models/error.rs',
  'sidebar/feed_list/models/feed.rs',
  'sidebar/feed_list/models/item.rs',
  'sidebar/feed_list/models/mod.rs',
  'sidebar/feed_list/models/tree.rs',
  'sidebar/feed_list/category_row.rs',
  'sidebar/feed_list/error.rs',
  'sidebar/feed_list/feed_row.rs',
  'sidebar/feed_list/mod.rs',
  'sidebar/footer/mod.rs',
  'sidebar/models/mod.rs',
  'sidebar/models/sidebar_iterate_item.rs',
  'sidebar/models/sidebar_selection.rs',
  'sidebar/tag_list/models/change_set.rs',
  'sidebar/tag_list/models/error.rs',
  'sidebar/tag_list/models/mod.rs',
  'sidebar/tag_list/models/tag.rs',
  'sidebar/tag_list/error.rs',
  'sidebar/tag_list/mod.rs',
  'sidebar/tag_list/tag_row.rs',
  'sidebar/error.rs',
  'sidebar/mod.rs',
  'undo_bar/models/mod.rs',
  'undo_bar/mod.rs',
  'util/builder_helper.rs',
  'util/date_util.rs',
  'util/error.rs',
  'util/file_util.rs',
  'util/gtk_util.rs',
  'util/mod.rs',
  'welcome_screen/mod.rs',
  'welcome_screen/service_row.rs',
  'welcome_screen/welcome_page.rs',
  'about_dialog.rs',
  'add_dialog.rs',
  'app.rs',
  'config.rs',
  'error_bar.rs',
  'error_dialog.rs',
  'i18n.rs',
  'main.rs',
  'main_window.rs',
  'main_window_state.rs',
  'rename_dialog.rs',
  'reset_page.rs',
  'responsive.rs'
)

features = ''
if get_option('readability')
  features += 'readability'
endif

cargo_release = custom_target('cargo-build',
                        build_by_default: true,
                        input: [
                                newsflash_sources,
                               ],
                        output: ['com.gitlab.newsflash'],
                        install: true,
                        install_dir: newsflash_bindir,
                        console: true,
                        command: [cargo_script,
                                  '@SOURCE_ROOT@',
                                  '@OUTPUT@',
                                  meson.build_root(),
                                  profile,
                                  '--features "@0@"'.format(features)
                                 ])
