#include "files.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QQmlEngine>

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

void FilesPlugin::registerTypes(const char *uri) {
    qmlRegisterSingletonType<Files>(uri, 1, 0, "Files",
                                    [](QQmlEngine *, QJSEngine *) -> QObject * { return new Files; });
}

#include "moc_files.cpp"
