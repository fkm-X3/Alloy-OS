#ifndef ALLOYMOUSE_H
#define ALLOYMOUSE_H

#include <QtCore/QObject>
#include <QtCore/QPoint>
#include <QtCore/Qt>

QT_BEGIN_NAMESPACE

class QAlloyMouse : public QObject
{
    Q_OBJECT

public:
    explicit QAlloyMouse(QObject *parent = nullptr);

    void handleMotion(int x, int y);
    void handleButton(int button, int state, int x, int y);
    void handleAxis(int axis, int value);
    void handleEnter(int surfaceId, int x, int y);
    void handleLeave(int surfaceId);

    int focusedSurface() const { return m_focusedSurface; }
    QPoint pos() const { return m_pos; }

private:
    Qt::MouseButtons translateButtons() const;
    Qt::MouseButton translateButton(int waylandButton) const;

    int m_focusedSurface;
    QPoint m_pos;
    bool m_buttons[3];
};

QT_END_NAMESPACE

#endif
