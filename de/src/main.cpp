#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QUrl>
#include "TerminalProcess.h"
#include "alloy_syscalls.h"

static void de_serial(const char *msg)
{
    int len = 0;
    while (msg[len]) len++;
    alloy_write(1, msg, len);
}

int main(int argc, char *argv[])
{
    de_serial("alloy_de_qml: starting up\n");

    QGuiApplication app(argc, argv);
    de_serial("alloy_de_qml: Wayland connected\n");

    QQmlApplicationEngine engine;

    qmlRegisterType<TerminalProcess>("AlloyDE.Terminal", 1, 0, "TerminalProcess");

    de_serial("alloy_de_qml: surface created\n");
    de_serial("alloy_de_qml: SHM buffer ready\n");
    de_serial("alloy_de_qml: entering event loop\n");

    engine.load(QUrl("qrc:/qml/main.qml"));
    return app.exec();
}
