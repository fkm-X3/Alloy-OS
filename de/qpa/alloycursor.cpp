#include "alloycursor.h"

QT_BEGIN_NAMESPACE

QAlloyCursor::QAlloyCursor()
    : QPlatformCursor()
{
}

void QAlloyCursor::changeCursor(QCursor *windowCursor, QWindow *window)
{
    // Alloy OS uses a server-side cursor managed by the compositor.
    // Future: send wl_pointer.set_cursor with a surface containing
    // the cursor bitmap from QCursor.
    Q_UNUSED(windowCursor);
    Q_UNUSED(window);
}

QPoint QAlloyCursor::pos() const
{
    // Return cached position from input state (future)
    return QPoint(0, 0);
}

void QAlloyCursor::setPos(const QPoint &pos)
{
    // Future: send pointer warp request to compositor
    Q_UNUSED(pos);
}

QT_END_NAMESPACE
