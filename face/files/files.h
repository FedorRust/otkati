#pragma once

#include <QObject>
#include <QString>
#include <QStringList>
#include <QtQml/QQmlExtensionPlugin>

class Files : public QObject {
    Q_OBJECT

public:
    explicit Files(QObject *parent = nullptr) : QObject(parent) {}

    Q_INVOKABLE QString readText(const QString &path) const;
    Q_INVOKABLE bool writeText(const QString &path, const QString &text) const;
    Q_INVOKABLE QStringList jsonFiles(const QString &directory) const;
    // One line to the ring, one line back. Empty when the ring is not listening.
    Q_INVOKABLE QString ask(const QString &line) const;
};

class FilesPlugin : public QQmlExtensionPlugin {
    Q_OBJECT
    Q_PLUGIN_METADATA(IID QQmlEngineExtensionInterface_iid)

public:
    void registerTypes(const char *uri) override;
};
