#include <QGuiApplication>
#include <QtQml/QQmlApplicationEngine>
#include <QtPlugin>

Q_IMPORT_PLUGIN(AlloyIntegrationPlugin)

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine;
    engine.loadData("import QtQuick 2.15\nWindow { width: 200; height: 200; visible: true; Rectangle { color: \"blue\"; anchors.fill: parent } }");
    return app.exec();
}
