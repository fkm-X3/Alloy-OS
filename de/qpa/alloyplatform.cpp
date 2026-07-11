#include "alloyplatform.h"
#include "alloystcreen.h"
#include "alloywindow.h"
#include "alloybackingstore.h"
#include "alloycursor.h"
#include "alloykeyboard.h"
#include "alloymouse.h"

#include <QtGui/qpa/qwindowsysteminterface.h>
#include <QtGui/qpa/qplatformfontdatabase.h>
#include <QtGui/private/qguiapplication_p.h>
#include <QtGui/private/qgenericunixeventdispatcher_p.h>
#include <QSocketNotifier>
#include <QEventLoopLocker>
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
    , m_keyboard(nullptr)
    , m_mouse(nullptr)
    , m_socketNotifier(nullptr)
    , m_eventLoopLocker(nullptr)
{
    s_instance = this;

    m_primaryScreen = new QAlloyScreen();
    QWindowSystemInterface::handleScreenAdded(m_primaryScreen);
}

QAlloyIntegration::~QAlloyIntegration()
{
    delete m_socketNotifier;
    delete m_eventLoopLocker;
    delete m_keyboard;
    delete m_mouse;
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
    const_cast<QAlloyIntegration *>(this)->registerSurface(surfaceId, w);
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

// --- C callback shims ---

extern "C" {

static void onKeyCb(int key, int pressed, struct input_state *state)
{
    Q_UNUSED(state);
    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (integration && integration->keyboard())
        integration->keyboard()->handleKey(key, pressed);
}

static void onKeyboardEnterCb(int surfaceId, struct input_state *state)
{
    Q_UNUSED(state);
    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (integration && integration->keyboard())
        integration->keyboard()->handleEnter(surfaceId);
}

static void onKeyboardLeaveCb(int surfaceId, struct input_state *state)
{
    Q_UNUSED(state);
    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (integration && integration->keyboard())
        integration->keyboard()->handleLeave(surfaceId);
}

static void onMouseMoveCb(int x, int y, struct input_state *state)
{
    Q_UNUSED(state);
    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (integration && integration->mouse())
        integration->mouse()->handleMotion(x, y);
}

static void onMouseEnterCb(int surfaceId, int x, int y, struct input_state *state)
{
    Q_UNUSED(state);
    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (integration && integration->mouse())
        integration->mouse()->handleEnter(surfaceId, x, y);
}

static void onMouseLeaveCb(int surfaceId, struct input_state *state)
{
    Q_UNUSED(state);
    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (integration && integration->mouse())
        integration->mouse()->handleLeave(surfaceId);
}

static void onClickCb(int button, int pressed, int x, int y,
                      struct input_state *state)
{
    Q_UNUSED(state);
    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (integration && integration->mouse())
        integration->mouse()->handleButton(button, pressed, x, y);
}

static void onAxisCb(int axis, int value, struct input_state *state)
{
    Q_UNUSED(state);
    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (integration && integration->mouse())
        integration->mouse()->handleAxis(axis, value);
}

} // extern "C"

int QAlloyIntegration::displayFd() const
{
    struct wl_display *d = static_cast<struct wl_display *>(m_display);
    return d ? d->fd : -1;
}

static void processWaylandEvents()
{
    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (!integration)
        return;
    struct wl_display *d = static_cast<struct wl_display *>(integration->display());
    if (d)
        wl_display_dispatch_pending(d);
}

void QAlloyIntegration::setupWaylandInput()
{
    struct wl_display *d = static_cast<struct wl_display *>(m_display);
    if (!d || !d->seat_registry_name)
        return;

    wl_set_key_callback(d, onKeyCb);
    wl_set_keyboard_enter_callback(d, onKeyboardEnterCb);
    wl_set_keyboard_leave_callback(d, onKeyboardLeaveCb);
    wl_set_mouse_move_callback(d, onMouseMoveCb);
    wl_set_mouse_enter_callback(d, onMouseEnterCb);
    wl_set_mouse_leave_callback(d, onMouseLeaveCb);
    wl_set_click_callback(d, onClickCb);
    wl_set_axis_callback(d, onAxisCb);

    unsigned int seatId = wl_seat_bind(
        static_cast<struct wl_registry *>(m_registry),
        d->seat_registry_name, d->seat_registry_version);
    if (!seatId)
        return;

    wl_seat_get_keyboard(d, seatId);
    wl_seat_get_pointer(d, seatId);

    m_keyboard = new QAlloyKeyboard(nullptr);
    m_mouse = new QAlloyMouse(nullptr);
}

void QAlloyIntegration::initialize()
{
    struct wl_display *d = wl_display_connect("/tmp/wayland-0");
    m_display = d;
    if (!d)
        return;

    struct wl_registry *reg = wl_display_get_registry(d);
    m_registry = reg;
    if (!reg)
        return;

    wl_display_dispatch_pending(d);

    m_compositorId = 3;

    setupWaylandInput();

    m_socketNotifier = new QSocketNotifier(d->fd, QSocketNotifier::Read, nullptr);
    QObject::connect(m_socketNotifier, &QSocketNotifier::activated, [this](int) {
        processWaylandEvents();
    });

    m_eventLoopLocker = new QEventLoopLocker();
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

void QAlloyIntegration::registerSurface(unsigned int surfaceId, QAlloyWindow *window)
{
    m_surfaceMap.insert(surfaceId, window);
}

void QAlloyIntegration::unregisterSurface(unsigned int surfaceId)
{
    m_surfaceMap.remove(surfaceId);
}

QWindow *QAlloyIntegration::windowForSurface(unsigned int surfaceId) const
{
    QAlloyWindow *w = m_surfaceMap.value(surfaceId, nullptr);
    return w ? w->window() : nullptr;
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
