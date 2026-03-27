"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { ChevronRight } from "lucide-react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarMenuSub,
  SidebarMenuSubItem,
  SidebarMenuSubButton,
} from "@/components/ui/sidebar";
import { type NavGroup, type NavLeaf, isNavLeaf } from "@/lib/navigation";

function isGroupActive(group: NavGroup, pathname: string): boolean {
  return group.children.some((child) =>
    isNavLeaf(child) ? child.href === pathname : isGroupActive(child, pathname)
  );
}

function NavTreeLeaf({ node, nested }: { node: NavLeaf; nested?: boolean }) {
  const pathname = usePathname();
  const isActive = pathname === node.href;

  if (nested) {
    return (
      <SidebarMenuSubItem>
        <SidebarMenuSubButton
          render={<Link href={node.href} />}
          isActive={isActive}
        >
          {node.label}
        </SidebarMenuSubButton>
      </SidebarMenuSubItem>
    );
  }

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        render={<Link href={node.href} />}
        isActive={isActive}
      >
        {node.label}
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
}

function NavTreeGroup({ node }: { node: NavGroup }) {
  const pathname = usePathname();
  const [open, setOpen] = useState(isGroupActive(node, pathname));

  return (
    <Collapsible open={open} onOpenChange={setOpen} className="group/collapsible">
      <SidebarMenuItem>
        <SidebarMenuButton render={<CollapsibleTrigger />}>
          <span className="font-medium">{node.label}</span>
          <ChevronRight className="ml-auto h-4 w-4 shrink-0 transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
        </SidebarMenuButton>
        <CollapsibleContent>
          <SidebarMenuSub>
            {node.children.map((child) =>
              isNavLeaf(child) ? (
                <NavTreeLeaf key={child.href} node={child} nested />
              ) : (
                <NavTreeGroup key={child.label} node={child} />
              )
            )}
          </SidebarMenuSub>
        </CollapsibleContent>
      </SidebarMenuItem>
    </Collapsible>
  );
}

export function NavTree({ items }: { items: (NavGroup | NavLeaf)[] }) {
  return (
    <SidebarMenu>
      {items.map((item) =>
        isNavLeaf(item) ? (
          <NavTreeLeaf key={item.href} node={item} />
        ) : (
          <NavTreeGroup key={item.label} node={item} />
        )
      )}
    </SidebarMenu>
  );
}
