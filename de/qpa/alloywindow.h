#ifndef ALLOYWINDOW_H
#define ALLOYWINDOW_H

#include <QtGui/qpa/qplatformwindow.h>

QT_BEGIN_NAMESPACE

class QAlloyWindow : public QPlatformWindow
{
public:
    QAlloyWindow(QWindow *window, unsigned int surfaceId);

    void setGeometry(const QRect &rect) override;
    void setVisible(bool visible) override;
    void setWindowTitle(const QString &title) override;
    void requestActivateWindow() override;
    WId winId() const override;

    unsigned int surfaceId() const { return m_surfaceId; }

private:
    unsigned int m_surfaceId;
};

QT_END_NAMESPACE

#endif
