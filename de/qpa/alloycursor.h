#ifndef ALLOYCURSOR_H
#define ALLOYCURSOR_H

#include <QtGui/qpa/qplatformcursor.h>

QT_BEGIN_NAMESPACE

class QAlloyCursor : public QPlatformCursor
{
public:
    QAlloyCursor();

    void changeCursor(QCursor *windowCursor, QWindow *window) override;
    QPoint pos() const override;
    void setPos(const QPoint &pos) override;
};

QT_END_NAMESPACE

#endif
