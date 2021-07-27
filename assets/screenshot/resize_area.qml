import QtQuick 2.0

Rectangle {

    enum Type {
        Left,
        Right,
        Top,
        Bottom,
        LeftTop,
        LeftBottom,
        RightTop,
        RightBottom
    }

    property int area_size: 18
    property color area_color: 'steelblue'
    property Type area_type

    width: area_size
    height: parent.height - width
    color: area_color()
    anchors.horizontalCenter: parent.left
    anchors.verticalCenter: parent.verticalCenter

    function area_width() {

    }

    function area_height() {

    }

    function area_color() {
        return is_edge() ? 'transparent' : area_color
    }

    function is_conrner() {
        return area_type === Type.LeftTop
                || area_type === Type.LeftBottom
                || area_type === Type.RightTop
                || area_type === Type.RightBottom
    }

    function is_edge(t) {
        return area_type === Type.Left
                || area_type === Type.Bottom
                || area_type === Type.Top
                || area_type === Type.Right
    }

    Rectangle {
        width: 2
        height: parent.height
        color: resize_area.color
        anchors.centerIn: parent
    }

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true

        onEntered: {
            cursorShape = Qt.SizeHorCursor
        }
        onExited: {
            cursorShape = Qt.ArrowCursor
        }

        drag {
            target: parent; axis: Drag.XAxis
        }
        onMouseXChanged: {
            if (!drag.active) {
                return
            }

            root.width = root.width - mouseX
            root.x = root.x + mouseX
            if(root.width < resize_area.size)
                root.width = resize_area.size
        }
    }
}
