#ifndef TERMINALPROCESS_H
#define TERMINALPROCESS_H

#include <QObject>
#include <QTimer>
#include <QString>

class TerminalProcess : public QObject
{
    Q_OBJECT
    Q_PROPERTY(bool running READ running NOTIFY runningChanged)

public:
    explicit TerminalProcess(QObject *parent = nullptr);
    ~TerminalProcess();

    bool running() const;

public slots:
    void start();
    void stop();
    void write(const QString &text);

signals:
    void outputReceived(const QString &text);
    void runningChanged();

private slots:
    void pollOutput();

private:
    int m_childPid;
    int m_stdinFd;
    int m_stdoutFd;
    int m_stderrFd;
    QTimer *m_pollTimer;
};

#endif
