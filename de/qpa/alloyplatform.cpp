#include "alloyplatform.h"
#include "alloystcreen.h"
#include "alloywindow.h"
#include "alloybackingstore.h"
#include "alloycursor.h"

#include <QtGui/qpa/qwindowsysteminterface.h>
#include <QtGui/qpa/qplatformfontdatabase.h>
#include <QtGui/private/qguiapplication_p.h>
#include <QtGui/private/qgenericunixeventdispatcher_p.h>
#include <QtPlugin>

#include <cstdint>

Q_IMPORT_PLUGIN(AlloyIntegrationPlugin)

extern "C" {
#include "wayland_client.h"
}

QT_BEGIN_NAMESPACE

class DummyFontDatabase : public QPlatformFontDatabase
{
public:
    void populateFontDatabase() override {}
};

static QAlloyIntegration *s_instance = nullptr;

QAlloyIntegration::QAlloyIntegration()
    : m_primaryScreen(nullptr)
    , m_cursor(nullptr)
    , m_display(nullptr)
    , m_registry(nullptr)
    , m_compositorId(0)
{
    s_instance = this;

    m_primaryScreen = new QAlloyScreen();
    QWindowSystemInterface::handleScreenAdded(m_primaryScreen);
}

QAlloyIntegration::~QAlloyIntegration()
{
    if (m_display)
        wl_display_disconnect(static_cast<struct wl_display *>(m_display));
    s_instance = nullptr;
}

bool QAlloyIntegration::hasCapability(Capability cap) const
{
    switch (cap) {
    case ThreadedPixmaps: return true;
    case MultipleWindows: return true;
    case WindowManagement: return true;
    case WindowActivation: return true;
    case NonFullScreenWindows: return true;
    case RasterGLSurface: return false;
    case RhiBasedRendering: return false;
    case PaintEvents: return true;
    default: return QPlatformIntegration::hasCapability(cap);
    }
}

QPlatformFontDatabase *QAlloyIntegration::fontDatabase() const
{
    static DummyFontDatabase *db = nullptr;
    if (!db)
        db = new DummyFontDatabase();
    return db;
}

QPlatformWindow *QAlloyIntegration::createPlatformWindow(QWindow *window) const
{
    unsigned int surfaceId = createSurfaceId();
    QAlloyWindow *w = new QAlloyWindow(window, surfaceId);
    return w;
}

QPlatformBackingStore *QAlloyIntegration::createPlatformBackingStore(QWindow *window) const
{
    return new QAlloyBackingStore(window);
}

QAbstractEventDispatcher *QAlloyIntegration::createEventDispatcher() const
{
    return createUnixEventDispatcher();
}

QPlatformNativeInterface *QAlloyIntegration::nativeInterface() const
{
    if (!m_nativeInterface)
        m_nativeInterface.reset(new QPlatformNativeInterface);
    return m_nativeInterface.get();
}

QPlatformCursor *QAlloyIntegration::createPlatformCursor() const
{
    if (!m_cursor)
        m_cursor = new QAlloyCursor();
    return m_cursor;
}

int QAlloyIntegration::displayFd() const
{
    struct wl_display *d = static_cast<struct wl_display *>(m_display);
    return d ? d->fd : -1;
}

void QAlloyIntegration::initialize()
{
    struct wl_display *d = wl_display_connect("/tmp/wayland.sock");
    m_display = d;
    if (!d)
        return;

    struct wl_registry *reg = wl_display_get_registry(d);
    m_registry = reg;
    if (!reg)
        return;

    wl_display_dispatch_pending(d);

    m_compositorId = 3;
}

unsigned int QAlloyIntegration::createSurfaceId() const
{
    struct wl_display *d = static_cast<struct wl_display *>(m_display);
    if (!d)
        return 0;
    unsigned int surfaceId = d->next_id++;
    wl_message_send(d->fd, m_compositorId,
                    WL_COMPOSITOR_CREATE_SURFACE,
                    &surfaceId, sizeof(surfaceId));
    return surfaceId;
}

QAlloyIntegration *QAlloyIntegration::instance()
{
    return s_instance;
}

QT_END_NAMESPACE

extern "C" QPlatformIntegration *createAlloyPlatformIntegration()
{
    return new QAlloyIntegration();
}
