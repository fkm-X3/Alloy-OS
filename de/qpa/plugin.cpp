#include "alloyplatform.h"
#include <qpa/qplatformintegrationplugin.h>
#include <QtPlugin>

QT_BEGIN_NAMESPACE

class QAlloyIntegrationPlugin : public QPlatformIntegrationPlugin
{
    Q_OBJECT
    Q_PLUGIN_METADATA(IID QPlatformIntegrationFactoryInterface_iid FILE "alloy.json")
public:
    QPlatformIntegration *create(const QString &key, const QStringList &paramList) override;
};

QPlatformIntegration *QAlloyIntegrationPlugin::create(const QString &key, const QStringList &paramList)
{
    Q_UNUSED(paramList);
    if (key.compare("alloy", Qt::CaseInsensitive) == 0)
        return createAlloyPlatformIntegration();
    return nullptr;
}

QT_END_NAMESPACE

#include "plugin.moc"
