#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QUrl>
#include "TerminalProcess.h"

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine;

    qmlRegisterType<TerminalProcess>("AlloyDE.Terminal", 1, 0, "TerminalProcess");

    engine.load(QUrl("qrc:/qml/main.qml"));
    return app.exec();
}
