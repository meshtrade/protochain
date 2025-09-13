'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { ChevronDownIcon, ChevronRightIcon } from '@heroicons/react/24/outline'
import { useState } from 'react'

interface NavItem {
  name: string
  href?: string
  children?: NavItem[]
  icon?: string
}

const navigation: NavItem[] = [
  {
    name: 'solana',
    children: [
      {
        name: 'account',
        children: [
          {
            name: 'v1',
            href: '/solana/account/v1',
          }
        ]
      },
      {
        name: 'transaction',
        children: [
          {
            name: 'v1',
            href: '/solana/transaction/v1',
          }
        ]
      },
      {
        name: 'program',
        children: [
          {
            name: 'system',
            children: [
              {
                name: 'v1',
                href: '/solana/program/system/v1',
              }
            ]
          },
          {
            name: 'token',
            children: [
              {
                name: 'v1',
                href: '/solana/program/token/v1',
              }
            ]
          }
        ]
      },
      {
        name: 'rpc_client',
        children: [
          {
            name: 'v1',
            href: '/solana/rpc_client/v1',
          }
        ]
      }
    ]
  }
]

interface NavItemComponentProps {
  item: NavItem
  level: number
}

function NavItemComponent({ item, level }: NavItemComponentProps) {
  const pathname = usePathname()
  const [isExpanded, setIsExpanded] = useState(
    item.children?.some(child => 
      child.href === pathname || 
      child.children?.some(grandchild => grandchild.href === pathname)
    ) || false
  )

  const isActive = item.href === pathname
  const hasChildren = item.children && item.children.length > 0

  const handleToggle = () => {
    if (hasChildren) {
      setIsExpanded(!isExpanded)
    }
  }

  const indentClass = `pl-${level * 4}`
  const baseClasses = `
    group flex items-center px-2 py-2 text-sm font-medium rounded-md cursor-pointer
    transition-colors duration-150 ease-in-out
  `
  
  const activeClasses = isActive
    ? 'bg-blue-100 text-blue-900 border-r-2 border-blue-600'
    : 'text-slate-700 hover:text-slate-900 hover:bg-slate-50'

  const content = (
    <div
      className={`${baseClasses} ${activeClasses} ${indentClass}`}
      onClick={handleToggle}
    >
      {hasChildren && (
        <span className="mr-2 h-4 w-4 flex-shrink-0">
          {isExpanded ? (
            <ChevronDownIcon className="h-4 w-4" />
          ) : (
            <ChevronRightIcon className="h-4 w-4" />
          )}
        </span>
      )}
      {!hasChildren && <span className="mr-6 h-4 w-4 flex-shrink-0" />}
      
      <span className="flex-1 font-mono text-xs">
        {item.name}
        {item.href && (
          <span className="text-slate-400 ml-1">
            — {item.name === 'v1' ? 'Service Methods' : 'API Endpoint'}
          </span>
        )}
      </span>
    </div>
  )

  return (
    <div>
      {item.href ? (
        <Link href={item.href} className="block">
          {content}
        </Link>
      ) : (
        content
      )}
      
      {hasChildren && isExpanded && (
        <div className="mt-1">
          {item.children?.map((child, index) => (
            <NavItemComponent
              key={`${child.name}-${index}`}
              item={child}
              level={level + 1}
            />
          ))}
        </div>
      )}
    </div>
  )
}

export default function Sidebar() {

  return (
    <div className="flex flex-col w-64 bg-white border-r border-slate-200">
      {/* Sidebar Header */}
      <div className="flex items-center h-16 px-4 border-b border-slate-200">
        <div className="flex items-center">
          <div className="h-8 w-8 bg-gradient-to-r from-purple-500 to-blue-600 rounded-lg flex items-center justify-center">
            <span className="text-white font-bold text-sm">P</span>
          </div>
          <div className="ml-3">
            <h1 className="text-lg font-semibold text-slate-900">Protochain</h1>
            <p className="text-xs text-slate-500">API Dashboard</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-4 py-4 space-y-1 overflow-y-auto">
        <div className="pb-2">
          <h2 className="text-xs font-semibold text-slate-400 uppercase tracking-wide">
            Protocol Services
          </h2>
        </div>
        
        {navigation.map((item, index) => (
          <NavItemComponent
            key={`${item.name}-${index}`}
            item={item}
            level={0}
          />
        ))}
      </nav>

      {/* Footer */}
      <div className="flex-shrink-0 p-4 border-t border-slate-200">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <div className="h-2 w-2 bg-green-400 rounded-full" />
          </div>
          <div className="ml-3">
            <p className="text-xs text-slate-500">
              Backend: {process.env.NODE_ENV === 'development' ? 'localhost:50051' : 'Connected'}
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}