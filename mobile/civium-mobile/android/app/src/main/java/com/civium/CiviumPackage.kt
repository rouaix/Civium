package com.civium

import com.facebook.react.ReactPackage
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.uimanager.ViewManager

class CiviumPackage : ReactPackage {
    override fun createNativeModules(context: ReactApplicationContext) =
        listOf(CiviumModule(context))

    override fun createViewManagers(context: ReactApplicationContext): List<ViewManager<*, *>> =
        emptyList()
}
