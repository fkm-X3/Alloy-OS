#ifndef ALLOYPLATFORM_H
#define ALLOYPLATFORM_H

#include <QtGui/qpa/qplatformintegration.h>
#include <QtGui/qpa/qplatformscreen.h>
#include <QtGui/qpa/qplatformnativeinterface.h>

QT_BEGIN_NAMESPACE

class QAlloyScreen;
class QAlloyCursor;

class QAlloyIntegration : public QPlatformIntegration
{
public:
    QAlloyIntegration();
    ~QAlloyIntegration();

    bool hasCapability(Capability cap) const override;
    QPlatformFontDatabase *fontDatabase() const override;
    QPlatformWindow *createPlatformWindow(QWindow *window) const override;
    QPlatformBackingStore *createPlatformBackingStore(QWindow *window) const override;
    QAbstractEventDispatcher *createEventDispatcher() const override;
    QPlatformNativeInterface *nativeInterface() const override;
    QPlatformCursor *createPlatformCursor() const;
    void initialize() override;

    int displayFd() const;
    unsigned int compositorId() const { return m_compositorId; }
    unsigned int createSurfaceId() const;

    static QAlloyIntegration *instance();

private:
    QAlloyScreen *m_primaryScreen;
    mutable QAlloyCursor *m_cursor;
    mutable void *m_display;
    mutable void *m_registry;
    mutable unsigned int m_compositorId;
    mutable QScopedPointer<QPlatformNativeInterface> m_nativeInterface;
};

QT_END_NAMESPACE

extern "C" QPlatformIntegration *createAlloyPlatformIntegration();

#endif
