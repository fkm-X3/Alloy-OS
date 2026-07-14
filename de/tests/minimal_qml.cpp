#include <QGuiApplication>
#include <QtQml/QQmlApplicationEngine>
#include <QtPlugin>
#include <unistd.h>

Q_IMPORT_PLUGIN(AlloyIntegrationPlugin)

static void debug_msg(const char *msg) {
    write(1, msg, __builtin_strlen(msg));
}

int main(int argc, char *argv[])
{
    debug_msg("[test_qml] main() entered\n");
    QGuiApplication app(argc, argv);
    debug_msg("[test_qml] QGuiApplication created\n");
    QQmlApplicationEngine engine;
    debug_msg("[test_qml] QQmlApplicationEngine created\n");
    engine.loadData("import QtQuick 2.15\nWindow { width: 200; height: 200; visible: true; Rectangle { color: \"blue\"; anchors.fill: parent } }");
    debug_msg("[test_qml] loadData called, entering app.exec()\n");
    int ret = app.exec();
    debug_msg("[test_qml] app.exec() returned\n");
    return ret;
}
