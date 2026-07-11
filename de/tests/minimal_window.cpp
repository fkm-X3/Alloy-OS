#include <QGuiApplication>
#include <QWindow>
#include <QtPlugin>

Q_IMPORT_PLUGIN(AlloyIntegrationPlugin)

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    QWindow w;
    w.resize(400, 300);
    w.setTitle("Hello Alloy");
    w.show();
    return app.exec();
}
