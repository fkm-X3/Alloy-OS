#include "TerminalProcess.h"
#include "alloy_syscalls.h"
#include <QtCore/QCoreApplication>

TerminalProcess::TerminalProcess(QObject *parent)
    : QObject(parent)
    , m_childPid(-1)
    , m_stdinFd(-1)
    , m_stdoutFd(-1)
    , m_stderrFd(-1)
    , m_pollTimer(new QTimer(this))
{
    connect(m_pollTimer, &QTimer::timeout,
            this, &TerminalProcess::pollOutput);
}

TerminalProcess::~TerminalProcess()
{
    stop();
}

bool TerminalProcess::running() const
{
    return m_childPid > 0;
}

void TerminalProcess::start()
{
    if (running())
        return;

    int stdinPipe[2];
    int stdoutPipe[2];

    if (alloy_pipe(stdinPipe) != 0)
        return;
    if (alloy_pipe(stdoutPipe) != 0) {
        alloy_close(stdinPipe[0]);
        alloy_close(stdinPipe[1]);
        return;
    }

    m_childPid = alloy_fork();

    if (m_childPid == 0) {
        // Child process
        alloy_close(stdinPipe[1]);
        alloy_close(stdoutPipe[0]);

        alloy_dup2(stdinPipe[0], 0);
        alloy_dup2(stdoutPipe[1], 1);
        alloy_dup2(stdoutPipe[1], 2);

        alloy_close(stdinPipe[0]);
        alloy_close(stdoutPipe[1]);

        const char *shell = "/bin/sh";
        alloy_execve(shell);

        // exec failed
        alloy_close(0);
        alloy_close(1);
        alloy_close(2);
        alloy_syscall(ALLOY_SYS_EXIT, 1, 0, 0, 0, 0);
        for (;;) {}
    }

    // Parent process
    alloy_close(stdinPipe[0]);
    alloy_close(stdoutPipe[1]);

    m_stdinFd = stdinPipe[1];
    m_stdoutFd = stdoutPipe[0];
    m_stderrFd = -1;

    m_pollTimer->start(16);
    emit runningChanged();
}

void TerminalProcess::stop()
{
    if (m_childPid <= 0)
        return;

    alloy_kill(m_childPid, 15);
    alloy_waitpid(m_childPid, 0);

    if (m_stdinFd >= 0) {
        alloy_close(m_stdinFd);
        m_stdinFd = -1;
    }
    if (m_stdoutFd >= 0) {
        alloy_close(m_stdoutFd);
        m_stdoutFd = -1;
    }

    m_childPid = -1;
    m_pollTimer->stop();
    emit runningChanged();
}

void TerminalProcess::write(const QString &text)
{
    if (!running() || m_stdinFd < 0)
        return;

    QByteArray utf8 = text.toUtf8();
    // Append newline for shell line-buffered mode
    utf8.append('\n');
    alloy_write(m_stdinFd, utf8.constData(), utf8.size());
}

void TerminalProcess::pollOutput()
{
    if (!running())
        return;

    char buf[512];
    for (int fd : {m_stdoutFd, m_stderrFd}) {
        if (fd < 0)
            continue;
        for (;;) {
            int n = alloy_read(fd, buf, sizeof(buf));
            if (n <= 0)
                break;
            QString data = QString::fromUtf8(buf, n);
            if (!data.isEmpty())
                emit outputReceived(data);
        }
    }

    // Check if child has exited
    int result = alloy_waitpid(m_childPid, 1); // WNOHANG = 1
    if (result != 0 && result != -1) {
        m_pollTimer->stop();
        if (m_stdinFd >= 0) {
            alloy_close(m_stdinFd);
            m_stdinFd = -1;
        }
        if (m_stdoutFd >= 0) {
            alloy_close(m_stdoutFd);
            m_stdoutFd = -1;
        }
        m_childPid = -1;
        emit runningChanged();
    }
}
