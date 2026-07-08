#ifndef ALLOYPLATFORM_H
#define ALLOYPLATFORM_H

#include <QtGui/qpa/qplatformintegration.h>
#include <QtGui/qpa/qplatformscreen.h>
#include <QtGui/qpa/qplatformnativeinterface.h>
#include <QtCore/QHash>
#include <QEventLoopLocker>

class QSocketNotifier;

QT_BEGIN_NAMESPACE

class QAlloyScreen;
class QAlloyCursor;
class QAlloyWindow;
class QAlloyKeyboard;
class QAlloyMouse;

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
    void *display() const { return m_display; }
    unsigned int compositorId() const { return m_compositorId; }
    unsigned int createSurfaceId() const;

    void registerSurface(unsigned int surfaceId, QAlloyWindow *window);
    void unregisterSurface(unsigned int surfaceId);
    QWindow *windowForSurface(unsigned int surfaceId) const;

    QAlloyKeyboard *keyboard() const { return m_keyboard; }
    QAlloyMouse *mouse() const { return m_mouse; }

    static QAlloyIntegration *instance();

private:
    void setupWaylandInput();

    QAlloyScreen *m_primaryScreen;
    mutable QAlloyCursor *m_cursor;
    mutable void *m_display;
    mutable void *m_registry;
    mutable unsigned int m_compositorId;
    mutable QScopedPointer<QPlatformNativeInterface> m_nativeInterface;
    QAlloyKeyboard *m_keyboard;
    QAlloyMouse *m_mouse;
    QSocketNotifier *m_socketNotifier;
    QEventLoopLocker *m_eventLoopLocker;
    mutable QHash<unsigned int, QAlloyWindow *> m_surfaceMap;
};

QT_END_NAMESPACE

extern "C" QPlatformIntegration *createAlloyPlatformIntegration();

#endif
