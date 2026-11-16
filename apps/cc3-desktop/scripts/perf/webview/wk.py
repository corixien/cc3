import gi,sys
gi.require_version('Gtk','3.0'); gi.require_version('WebKit2','4.1')
from gi.repository import Gtk, WebKit2, GLib
w=Gtk.Window(); w.set_default_size(1400,900)
v=WebKit2.WebView(); w.add(v); w.show_all()
v.load_uri('file://'+__import__('os').path.abspath(__import__('os').path.dirname(__file__))+'/bench.html')
def poll():
    t=v.get_title() or ''
    if t.startswith('RESULT'):
        print('webkitgtk',t[7:]); Gtk.main_quit(); return False
    return True
GLib.timeout_add(500,poll); GLib.timeout_add(120000,Gtk.main_quit)
Gtk.main()
