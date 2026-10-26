"""Ask the system which screen to give the program.

Prints the chosen output name, or nothing when the person cancels.
"""

import secrets
import sys

from gi.repository import Gio, GLib


def sender_token(bus):
    return bus.get_unique_name()[1:].replace(".", "_")


def request_path(sender, token):
    return f"/org/freedesktop/portal/desktop/request/{sender}/{token}"


def call(bus, proxy, method, params, token, sender):
    path = request_path(sender, token)
    loop = GLib.MainLoop()
    box = {}

    def on_signal(_conn, _sender, spath, _iface, signal, parameters):
        if signal != "Response" or spath != path:
            return
        response, body = parameters.unpack()
        box["response"] = response
        box["body"] = body
        if loop.is_running():
            loop.quit()

    sub = bus.signal_subscribe(
        None,
        "org.freedesktop.portal.Request",
        "Response",
        path,
        None,
        Gio.DBusSignalFlags.NONE,
        on_signal,
    )
    try:
        proxy.call_sync(method, params, Gio.DBusCallFlags.NONE, GLib.MAXINT, None)
        if "response" not in box:
            loop.run()
    finally:
        bus.signal_unsubscribe(sub)
    if "response" not in box:
        raise SystemExit("the system question did not answer")
    return box["response"], box["body"]


def chosen_name(body):
    streams = body.get("streams") or []
    for _node, props in streams:
        name = props.get("mapping_id")
        if name:
            return name
    return ""


def main():
    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    sender = sender_token(bus)
    portal = Gio.DBusProxy.new_sync(
        bus,
        Gio.DBusProxyFlags.NONE,
        None,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        "org.freedesktop.portal.ScreenCast",
        None,
    )
    session_token = "otkati" + secrets.token_hex(4)
    create_token = "otkati" + secrets.token_hex(4)
    response, body = call(
        bus,
        portal,
        "CreateSession",
        GLib.Variant(
            "(a{sv})",
            (
                {
                    "handle_token": GLib.Variant("s", create_token),
                    "session_handle_token": GLib.Variant("s", session_token),
                },
            ),
        ),
        create_token,
        sender,
    )
    if response != 0:
        return
    session = body["session_handle"]
    select_token = "otkati" + secrets.token_hex(4)
    response, _body = call(
        bus,
        portal,
        "SelectSources",
        GLib.Variant(
            "(oa{sv})",
            (
                session,
                {
                    "handle_token": GLib.Variant("s", select_token),
                    "types": GLib.Variant("u", 1),
                    "multiple": GLib.Variant("b", False),
                    "cursor_mode": GLib.Variant("u", 2),
                    "persist_mode": GLib.Variant("u", 2),
                },
            ),
        ),
        select_token,
        sender,
    )
    if response != 0:
        return
    start_token = "otkati" + secrets.token_hex(4)
    response, body = call(
        bus,
        portal,
        "Start",
        GLib.Variant(
            "(osa{sv})",
            (
                session,
                "",
                {"handle_token": GLib.Variant("s", start_token)},
            ),
        ),
        start_token,
        sender,
    )
    name = chosen_name(body) if response == 0 else ""
    session_proxy = Gio.DBusProxy.new_sync(
        bus,
        Gio.DBusProxyFlags.NONE,
        None,
        "org.freedesktop.portal.Desktop",
        session,
        "org.freedesktop.portal.Session",
        None,
    )
    try:
        session_proxy.call_sync("Close", None, Gio.DBusCallFlags.NONE, 2000, None)
    except GLib.Error:
        pass
    if name:
        sys.stdout.write(name + "\n")


if __name__ == "__main__":
    main()
