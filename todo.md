# TODO task

pallet_ibc里加一个create_router和DefaultRouter， DefaultRouter里没有其他ibc applications
pallet_ibc里加一个关联类型router, 在runtime/libs.rs里配置 router = DefaultRouter

当用户在substrate里集成了pallet_ibc_transfer后，先runtime/libs.rs对pallet_ibc_transfer配置
create_router或在default router上add_module(pallet_ibc_transfer::Module)
ModuleId直接获取pallet_ibc_transfer::Module里的值
这样组合


要解耦pallet_ibc和pallet_ibc_application
前者不知道后者，后者也不知道前者
但是后者是一个ibc/core/ics26_routing/context::Module

pallet_ibc里的router可以 add ibc/core/ics26_routing/context::Module
