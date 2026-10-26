#include "files.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QQmlEngine>

#include <cstring>
#include <cerrno>
#include <csignal>
#include <poll.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

namespace {

QByteArray ringSocketPath() {
    const QByteArray given = qgetenv("OTKATI_SOCKET");
    if (!given.isEmpty())
        return given;
    const QByteArray runtime = qgetenv("XDG_RUNTIME_DIR");
    if (runtime.isEmpty())
        return {};
    return runtime + "/otkati/ring.sock";
}

bool writeAll(int fd, const QByteArray &payload) {
    qsizetype sent = 0;
    while (sent < payload.size()) {
        const ssize_t wrote = ::write(fd, payload.constData() + sent, static_cast<size_t>(payload.size() - sent));
        if (wrote < 0) {
            if (errno == EINTR)
                continue;
            return false;
        }
        sent += wrote;
    }
    return true;
}

}

QString Files::readText(const QString &path) const {
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly))
        return {};
    return QString::fromUtf8(file.readAll());
}

bool Files::writeText(const QString &path, const QString &text) const {
    const QFileInfo info(path);
    QDir().mkpath(info.absolutePath());
    QFile file(path);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Truncate))
        return false;
    const QByteArray bytes = text.toUtf8();
    return file.write(bytes) == bytes.size();
}

QStringList Files::jsonFiles(const QString &directory) const {
    const QDir dir(directory);
    if (!dir.exists())
        return {};
    const QStringList names = dir.entryList({QStringLiteral("*.json")}, QDir::Files, QDir::Name);
    QStringList paths;
    for (const QString &name : names)
        paths.append(dir.absoluteFilePath(name));
    return paths;
}

QString Files::ask(const QString &line) const {
    const QByteArray path = ringSocketPath();
    if (path.isEmpty() || path.size() >= static_cast<qsizetype>(sizeof(sockaddr_un::sun_path)))
        return {};
    const int fd = ::socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0)
        return {};
    sockaddr_un addr{};
    addr.sun_family = AF_UNIX;
    std::memcpy(addr.sun_path, path.constData(), static_cast<size_t>(path.size()));
    if (::connect(fd, reinterpret_cast<sockaddr *>(&addr), sizeof(addr)) != 0) {
        ::close(fd);
        return {};
    }
    QByteArray payload = line.toUtf8();
    payload.append('\n');
    if (!writeAll(fd, payload)) {
        ::close(fd);
        return {};
    }
    QByteArray reply;
    while (!reply.contains('\n') && reply.size() < 1024) {
        pollfd ready{};
        ready.fd = fd;
        ready.events = POLLIN;
        if (::poll(&ready, 1, 20000) <= 0)
            break;
        char byte = 0;
        const ssize_t read = ::read(fd, &byte, 1);
        if (read <= 0)
            break;
        reply.append(byte);
    }
    ::close(fd);
    return QString::fromUtf8(reply).trimmed();
}

void FilesPlugin::registerTypes(const char *uri) {
    // The face outlives the shell that started it. A hangup from that shell
    // is not the person closing the program.
    std::signal(SIGHUP, SIG_IGN);
    qmlRegisterSingletonType<Files>(uri, 1, 0, "Files",
                                    [](QQmlEngine *, QJSEngine *) -> QObject * { return new Files; });
}

#include "moc_files.cpp"
